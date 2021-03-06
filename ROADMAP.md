# TODOs / Fixes

 * comments, documentation
 * more tests...
 * development speed convenience features
   - script for reload with cargo watch
 * client: more event propagation
 * library for async desktop notifications


# Performance Optimizations - TODO

 * js: optimize patch decode
      - profile
      - closure compiler?!

 * parallel component rendering
 * key-based diffing
 * diff: hash based comparison
      - Merkle trees ?!
      - good hashfunction for the job? => DefaultHasher from rust seems ok


# Performance Optimizations - Done

 * Patch optimization:
    => nth-child navigation instruction
 * component based rendering
   => only rerender components which need it
 * establish better benchmarking solution
      - try to establish common usecases and implement benchmarks
      - find a good way to instrument code and collect data
 * serialization frontend <-> backend: JSON for normal messages, binary for patches seems to be the fastest solution
   - JSON.parse() in the browser is pretty quick and outperforms js implementations of better formats
   - initial testing showed that serialization/deserialization was about 80% of overall render time in a small-ish example
 * lazy rerender
 * don't back annotate ids, just keep translation and use when generating next patch
 * frame limiter
 * patch optimizations
   - don't emit unnecessary moves
   - truncate moves from end of patch
 * separate update() and render() thread



# Features

 * some webgl interface?!
 * find the best way to deploy:
   * some webview based deployment?
   * cef?
   * qt webengine
   * or indeed electron due to all the other nice features?!


# Notes

cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"

