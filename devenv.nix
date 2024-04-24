{ pkgs, lib, config, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = with pkgs; [ gdb git curl gzip gnuplot];

  # https://devenv.sh/scripts/
  scripts.prepare-tests = {
    exec = ''
      touch ./test/NY.distances
      RUST_BACKTRACE=1 cargo test -- sssp_test_binary
    '';
  };
  scripts.pull-data = {
    exec = ''
      if [[ ( $@ == "--help") ||  ($@ == "-h") || ($# -ne 1)]]
      then 
        echo "Usage: pull-data [NAME]

        Choose one of:

        Name | Description
        _____|_______________________
        USA  | Full USA 	
        CTR  | Central USA 	
        W 	 | Western USA 	
        E 	 | Eastern USA 	
        LKS  | Great Lakes 	
        CAL  | California and Nevada 	
        NE 	 | Northeast USA 	
        NW 	 | Northwest USA 	
        FLA  | Florida 	
        COL  | Colorado 	
        BAY  | San Francisco Bay Area 	
        NY 	 | New York City"
        exit 0
      fi 
      curl "https://www.diag.uniroma1.it/challenge9/data/USA-road-d/USA-road-d.$1.gr.gz" | gunzip > data/$1-d.gr
      curl "https://www.diag.uniroma1.it/challenge9/data/USA-road-t/USA-road-t.$1.gr.gz" | gunzip > data/$1-t.gr
      curl "https://www.diag.uniroma1.it/challenge9/data/USA-road-d/USA-road-d.$1.co.gz" | gunzip > data/$1.co
    '';
    description = "Pull graph data from https://www.diag.uniroma1.it/challenge9/download.shtml";
  };

  enterShell = ''
      echo
      echo 🦾 Helper scripts you can run to make your development richer:
      echo 
      ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|󰮺  |' -e 's|••| |g'
      ${lib.generators.toKeyValue {} (lib.mapAttrs (name: value: value.description) config.scripts)}
      EOF
      echo
    '';

  # https://devenv.sh/languages/
  languages.nix.enable = true;

  languages.rust.enable = true;
  languages.rust.channel = "nixpkgs";

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # https://devenv.sh/processes/
  # processes.ping.exec = "ping example.com";

  # See full reference at https://devenv.sh/reference/options/
}
