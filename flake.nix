{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      mkPackage =
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          manifest = (nixpkgs.lib.importTOML ./Cargo.toml).package;

          toolchain = fenix.packages.${system}.stable.toolchain;
          platform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          default = platform.buildRustPackage {
            pname = manifest.name;
            version = manifest.version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            postInstall = ''
              ln -s $out/bin/hexlify $out/bin/unhexlify           
            '';
          };
        };

      mkDevShell =
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          toolchain = fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "KUm16pHj+cRedf8vxs/Hd2YWxpOrWZ7UOrwhILdSJBU=";
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [ toolchain ];
          };
        };
    in
    {
      packages = forAllSystems mkPackage;
      devShells = forAllSystems mkDevShell;
    };
}
