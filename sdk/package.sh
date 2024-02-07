cd ./src/wasm/pkg
jq '. + {"type": "module"}' package.json > temp
mv temp package.json

echo 'import * as js from "./invariant_a0_wasm.js"' >> invariant_a0_wasm.js
echo 'export default js' >> invariant_a0_wasm.js