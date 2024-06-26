{
    description = "Trabalho de SSD";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
	};

    outputs = { self, ... } @ inputs: 
    let
        pkgs = import inputs.nixpkgs { inherit system; };
        devroot = builtins.getEnv "PWD";
        name = "Trabalho de SSD";
        system = "x86_64-linux";
    in {
        devShells."${system}".default = pkgs.mkShell {
            inherit name devroot;

            buildInputs = with pkgs; [
                cargo
                rustc
                rust-analyzer
                protobuf
                openssl
                killall
            ];
            OPENSSL_DIR="${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";

            shellHook = ''
                alias build='cargo build'
                alias run='cargo run'
                alias test='cargo test'
                alias doc='cargo doc'
                alias odoc='cargo doc --open'

                alias client='cargo client && killall -9 public_ledger'
                alias bootstrap='cargo bootstrap && killall -9 public_ledger'
                alias server='cargo server1 && killall -9 public_ledger'
                alias server3='cargo server3 && killall -9 public_ledger'
            '';

        };
    };
}
