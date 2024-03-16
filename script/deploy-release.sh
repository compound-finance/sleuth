#!/bin/bash

set -eo pipefail

tag="$1"

if [ -z "$tag" ]; then
  echo "usage script/deploy-release.sh <tag>"
  exit 1
fi

opts=""

if [ -n "$DEPLOYER_PK" ]; then
  opts="$opts --private-key $DEPLOYER_PK --broadcast"
fi

if [ -n "$ETHERSCAN_API_KEY" ]; then
  opts="$opts --verify --etherscan-api-key \"$ETHERSCAN_API_KEY\""
fi

cleanup() {
    rv=$?
    rm -rf .release-tmp
    exit $rv
}

trap "cleanup" EXIT

rm -rf .release-tmp
mkdir .release-tmp

curl -L "https://github.com/compound-finance/sleuth/releases/download/$tag/Sleuth.json" -o ./.release-tmp/Sleuth.json
curl -L "https://github.com/compound-finance/sleuth/releases/download/$tag/contracts.json" -o ./.release-tmp/contracts.json

if [ -z "$RPC_URL" ]; then
  echo "Missing RPC_URL env var"
  exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "jq could not be found"
    exit 1
fi

export SLEUTH_ADDRESS="$(cat ./.release-tmp/contracts.json | jq -r '.sleuth')"
export CODE_JAR="$(cat ./.release-tmp/contracts.json | jq -r '.codeJar')"

forge script \
  --rpc-url="$RPC_URL" \
  $opts \
  script/Sleuth.s.sol:Deploy \
  $@
