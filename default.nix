{ pkgs ? import <nixpkgs> {} }:
pkgs.rustPlatform.buildRustPackage {
  pname = "lockbook-cli";
  version = "0.1.0";

  # Path to the Rust workspace's root directory
  src = ./.;

  # Specify the binary to build within the workspace
  cargoLock = {
	lockFile = ./Cargo.lock;

	  outputHashes = {
		"lb-fonts-0.1.5" = "sha256-WPUj+vn/E7uZrYvrpDonHQawEggKu+nvE0CVBPbDtyM=";
		"lb-pdf-0.2.3" = "sha256-1Fj1qGxtZnUIwH+x2XZTXYufc+ECcr5/P+6ERuc8PvU=";
		"minidom-0.15.3" = "sha256-V3Xy7r3eBheMlvVpGC/M/lTS7sM0C3L7ATRoeuM5c2A=";
	  };
  };

  doCheck = false;
  cargoBuildFlags = [ "--package" "lockbook-cli" ];

  # Optional: Declare metadata
  meta = with pkgs.lib; {
    description = "Minimal viable nix build for a Rust workspace CLI";
    license = licenses.mit; # Replace with your license
    maintainers = [ maintainers.yourUsername ]; # Add your handle here
  };

  postInstall = ''
    # Generate and install shell completions
    mkdir -p $out/share/bash-completion/completions
    mkdir -p $out/share/zsh/site-functions
    mkdir -p $out/share/fish/vendor_completions.d

    # Generate the completion scripts
    $out/bin/lockbook completions bash > $out/share/bash-completion/completions/cli
    $out/bin/lockbook completions zsh > $out/share/zsh/site-functions/_cli
    $out/bin/lockbook completions fish > $out/share/fish/vendor_completions.d/cli.fish
  '';
}
