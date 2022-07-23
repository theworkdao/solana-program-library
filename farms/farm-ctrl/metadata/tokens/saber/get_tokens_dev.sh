#!/usr/bin/env bash

url="https://registry.saber.so/data/token-list.devnet.json"

if hash wget 2>/dev/null; then
  wget_or_curl="wget -O tokens_dev.json $url"
elif hash curl 2>/dev/null; then
  wget_or_curl="curl -o tokens_dev.json -L $url"
else
  echo "Error: Neither curl nor wget were found" >&2
  return 1
fi

exec $wget_or_curl