npm run contract:build &&
rm -rf target &&
cp ../target/ink/invariant.wasm ./contracts/invariant/ &&
cp ../target/ink/invariant.json ./contracts/invariant/ &&
cp ../src/token/target/ink/token.wasm ./contracts/psp22/psp22.wasm &&
cp ../src/token/target/ink/token.json ./contracts/psp22/psp22.json &&
touch src/abis/invariant.ts &&
echo "export const abi = \`" > ./src/abis/invariant.ts &&
cat ./contracts/invariant/invariant.json >> ./src/abis/invariant.ts &&
echo "\`" >> ./src/abis/invariant.ts &&
touch src/abis/psp22.ts &&
echo "export const abi = \`" > ./src/abis/psp22.ts &&
cat ./contracts/psp22/psp22.json >> ./src/abis/psp22.ts &&
echo "\`" >> ./src/abis/psp22.ts
