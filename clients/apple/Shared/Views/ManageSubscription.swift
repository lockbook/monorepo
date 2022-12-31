import SwiftUI
import AlertToast

struct ManageSubscription: View {    
    @EnvironmentObject var billing: BillingService
    @EnvironmentObject var settings: SettingsService
    
    @Environment(\.presentationMode) var presentationMode
            
    var body: some View {
        VStack(alignment: .leading) {
            usage
            trial
            
            Text("Expand your storage to **30GB** for **\(billing.maybeMonthlySubscription?.displayPrice ?? "$2.99")** a month.")
                .padding(.vertical)
            
            HStack {
                Spacer()
                Button("Subscribe")
                {
                    print("PRESSED")
                    billing.purchasePremium()
                }
                .buttonStyle(.borderedProminent)
                .font(.title2)
                .padding(.top)
                .disabled(billing.purchaseResult == .some(.inFlow))
                
                Spacer()
            }
            
            if case .some(.inFlow) = billing.purchaseResult {
                loading
            } else if case .some(.failure) = billing.purchaseResult {
                error
            }

            
            Spacer()
        }
            .padding()
            .navigationTitle("Premium")
            .toast(isPresenting: Binding(get: {
                billing.showPurchaseToast
            }, set: { _ in
                billing.showPurchaseToast = false
            }), duration: 2, tapToDismiss: true) {
                purchaseToast()
            }
            .onChange(of: billing.purchaseResult) { newValue in
                if case .some(.success) = newValue {
                    DispatchQueue.global(qos: .userInitiated).async {
                        Thread.sleep(forTimeInterval: 2)
                        DispatchQueue.main.async {
                            
                            presentationMode.wrappedValue.dismiss()
                        }
                    }
                }
            }
    }
    
    func purchaseToast() -> AlertToast {
        switch billing.purchaseResult {
        case .some(.success):
            return AlertToast(type: .regular, title: "You have successfully purchased premium!")
        case .some(.pending):
            return AlertToast(type: .regular, title: "Your purchase is pending.")
        default:
            return AlertToast(type: .regular, title: "ERROR")
        }
    }
    
    @ViewBuilder
    var usage: some View {
        VStack (alignment: .leading) {
            Text("Current Usage:")
            ColorProgressBar(value: settings.usageProgress)
        }
        .padding(.vertical)
    }
    
    @ViewBuilder
    var trial: some View {
        VStack(alignment: .leading) {
            Text("If you upgraded, your usage would be:")
            ColorProgressBar(value: settings.premiumProgress)
        }
    }
    
    @ViewBuilder
    var error: some View {
        HStack {
            Spacer()
            Text("Failed to complete purchase.")
                .padding(.vertical)
                .foregroundColor(.red)
            Spacer()
        }
    }
    
    @ViewBuilder
    var loading: some View {
        HStack {
            Spacer()
            ProgressView()
            Spacer()
        }
    }
}

struct ManageSubscriptionView_Previews: PreviewProvider {
    static var previews: some View {
        NavigationView {
            ManageSubscriptionView()
                .mockDI()
        }
    }
}
