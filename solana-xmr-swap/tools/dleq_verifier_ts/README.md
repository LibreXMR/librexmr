# DLEQ Verifier (TS Wrapper)

Minimal TypeScript wrapper around the Rust DLEQ verifier.

## Setup
```
npm install
npm run build
```

## Verify (default vector)
```
npm run verify
```

## Verify (custom input)
```
node dist/index.js --input ../../test_vectors/dleq.json --verbose
```

## Use a prebuilt binary
```
node dist/index.js --bin /path/to/dleq-verify --input ../../test_vectors/dleq.json
```
