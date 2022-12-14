#!/bin/bash

set -exo pipefail

# anvil --port 8599 &
# anvil_pid="$!"
# sleep 3

# if kill -0 "$anvil_pid"; then
#   echo "anvil running"
# else
#   echo "anvil failed"
#   wait "$anvil_pid"
# fi

# while ! nc -z localhost 8599; do
#   sleep 3
# done

# function cleanup {
#   kill "$anvil_pid"
# }

# trap cleanup EXIT

# forge build
# forge create --rpc-url http://localhost:8599 --private-key 0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6 src/Sleuth.sol:Sleuth

SLEUTH_DEPLOYER=0xa0ee7a142d267c1f36714e4a8f75612f20a79720 yarn test
