#!/bin/bash
#
# NOTE: THIS SCRIPT IS INTENDED TO BE RUN IN THE CONTAINER, NOT THE HOST.

echo ""
echo "============== RUNNING DECOMPILERMC =============="
echo ""

yes | python3 main.py --mcversion "${VERSION}"

echo ""
echo "======== COPYING SRC FILES TO HOST VOLUME ========"
echo ""

cp -R "src/${VERSION}" "${HOST_VOLUME}/"

echo "Done."
