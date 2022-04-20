﻿using Windows.UI.Xaml.Controls;

namespace lockbook {

    public sealed partial class Startup : Page {
        public string Title {
            get {
                return TitleTextBlock.Text;
            }
            set {
                TitleTextBlock.Text = value;
            }
        }

        public string Message {
            get {
                return MessageTextBlock.Text;
            }
            set {
                MessageTextBlock.Text = value;
            }
        }

        public bool Working {
            get {
                return WorkingProgressRing.IsActive;
            }
            set {
                WorkingProgressRing.IsActive = value;
            }
        }

        public void Refresh() {
            if (App.ClientUpdateRequired) {
                Working = false;
                Title = "Update Lockbook";
                Message = "Update required.";
            } else {
                Working = true;
                Title = "Loading";
                Message = "";
            }
        }

        public Startup() {
            InitializeComponent();
        }
    }
}
