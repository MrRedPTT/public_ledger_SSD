{
    description = "Trabalho de SSD";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
	};

    outputs = { self, ... } @ inputs: 
    let
        pkgs = import inputs.nixpkgs { inherit system; };
        root = builtins.getEnv "PWD";
        name = "Trabalho de SSD";
        system = "x86_64-linux";
    in {
        devShells."${system}".default = pkgs.mkShell {
            inherit name;

            buildInputs = with pkgs; [
                cargo
                rustc
            ];

            ROOT=root;

            shellHook = ''
                echo -ne "\033]0;${name}\007"

                alias build='cargo build'
                alias run='cargo run'

                echo rustc is @ version ${pkgs.rustc}
                echo cargo is @ version ${pkgs.cargo}
            '';

        };
    };
}
