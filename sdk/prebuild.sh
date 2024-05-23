npm run contract:build &&
rm -rf target &&
cp ../target/ink/invariant.wasm ./contracts/invariant/ &&
cp ../target/ink/invariant.json ./contracts/invariant/ &&
touch src/abis/invariant.ts &&
echo "export const abi = \`" > ./src/abis/invariant.ts &&
cat ./contracts/invariant/invariant.json >> ./src/abis/invariant.ts &&
echo "\`" >> ./src/abis/invariant.ts
