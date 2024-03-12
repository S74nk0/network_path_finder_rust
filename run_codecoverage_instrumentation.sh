# setup instrument-coverage
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
# export RUSTFLAGS="-Zunstable-options -C instrument-coverage=except-unused-generics"
# export RUSTFLAGS="-Zunstable-options -C instrument-coverage=except-unused-generics -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off "
# export RUSTFLAGS="-Zunstable-options -C instrument-coverage=except-unused-functions"
# export RUSTFLAGS="-Zunstable-options -C instrument-coverage=except-unused-functions --cfg feature=skip_tests_coverage"
export RUSTDOCFLAGS="-Cpanic=abort"

# build and run tests
cargo +nightly build --all
cargo +nightly test --all

# generate report
timestamp=`date +%s`
cov_path="coverage_inst_$timestamp"
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
