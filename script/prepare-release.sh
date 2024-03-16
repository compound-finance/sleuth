#!/bin/bash

set -eo pipefail

if [ -z "$CODE_JAR" ]; then
  echo "Missing CODE_JAR env var"
  exit 1
fi

if [ -z "$RPC_URL" ]; then
  echo "Missing RPC_URL env var"
  exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "jq could not be found"
    exit 1
fi

forge build
mkdir -p release/
cp out/Sleuth.sol/Sleuth.json release/
cp src/Sleuth.sol release/
title="$(git log -1 --pretty="%s")"
body="$(git log -1 --pretty="%b")"

if [ -z "$title" ]; then
  echo "must include git commit title"
  exit 1
fi

if [ -z "$body" ]; then
  echo "must include git commit body"
  exit 1
fi

sleuth_address="$(forge script --rpc-url="$RPC_URL" --json --silent script/Sleuth.s.sol:Prepare | tee | jq -r '.returns."0".value')"
forge verify-contract --show-standard-json-input 0x0000000000000000000000000000000000000000 src/Sleuth.sol:Sleuth > release/sleuth-verify.json

echo "title=$title"
echo "body=$body"
echo "sleuth_address=$sleuth_address"

echo "$sleuth_address" > "release/sleuth@$sleuth_address"

cat > release/RELEASE.md <<EOF
## $title

Sleuth Address: $sleuth_address

$body
EOF

cat > release/contracts.json <<EOF
{
  "sleuth": "$sleuth_address",
  "codeJar": "$CODE_JAR"
}
EOF
