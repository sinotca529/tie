RUSTFLAGS="-Z instrument-coverage" \
    LLVM_PROFILE_FILE="test.profraw" \
    cargo +nightly test --tests

cargo +nightly profdata -- merge \
    -sparse test.profraw -o test.profdata

cargo +nightly cov -- show \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-Z instrument-coverage" \
            cargo +nightly test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
    --use-color \
    --instr-profile=test.profdata \
    --ignore-filename-regex='(.cargo|rustc)' \
    --Xdemangler=rustfilt \
    --show-line-counts-or-regions \
    --show-instantiations \
    --format='html' \
    --output-dir='llvm-cov-report' \
