# Rust code cov setup
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"


# build and run tests set explicit target so we can bypass the panic stretegy for proc-macro2
cargo +nightly build --all --target x86_64-unknown-linux-gnu
cargo +nightly test --all --target x86_64-unknown-linux-gnu

# generate report
timestamp=`date +%s`
cov_path="coverage_$timestamp"
# # TODO deps or no deps genertion difference ????
# grcov . -s . --binary-path ./target/debug/deps/ -t html --branch --ignore-not-existing -o ./target/debug/$cov_path
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/$cov_path
# cleanup 
rm *.profraw
rm */*.profraw
rm */*/*.profraw

# under WSL copy to host
if grep -qi microsoft /proc/version; then
  if [ ! -d /mnt/c/tmp/ ]; then
    mkdir /mnt/c/tmp/
  fi
  echo "moving target/debug/$cov_path/ to /mnt/c/tmp/$cov_path/"
  mv target/debug/$cov_path/ /mnt/c/tmp/
fi

echo "$cov_path DONE!"
