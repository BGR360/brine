#!/bin/bash

SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

# Default values for variables
: ${VERSION:="1.14.4"}
: ${DESTDIR:="${SCRIPTPATH}/src"}
: ${DOCKER_IMAGE_NAME:="mc-decompile"}

set -e
set -x

docker build "${SCRIPTPATH}" -t "${DOCKER_IMAGE_NAME}"

SRCDIR="/src/"
DESTDIR="$(realpath ${DESTDIR})"

mkdir -p "${DESTDIR}"

docker run \
    --rm \
    --volume "${DESTDIR}:${SRCDIR}" \
    --env "VERSION=${VERSION}" \
    --env "HOST_VOLUME=${SRCDIR}" \
    "${DOCKER_IMAGE_NAME}" \
    "./decompile-and-copy.sh"
