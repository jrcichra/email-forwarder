#!/bin/bash
set -x

# env vars
export SHA="sha-${SHA::8}"
# pull the containers
docker pull ghcr.io/jrcichra/email-fanout:${SHA}-amd64
docker pull ghcr.io/jrcichra/email-fanout:${SHA}-arm64

#https://www.docker.com/blog/multi-arch-build-and-images-the-simple-way/

# update the current sha on the base image
docker manifest create ghcr.io/jrcichra/email-fanout:${SHA} \
    --amend ghcr.io/jrcichra/email-fanout:${SHA}-amd64 \
    --amend ghcr.io/jrcichra/email-fanout:${SHA}-arm64
docker manifest push ghcr.io/jrcichra/email-fanout:${SHA}

# if main branch, update the latest tag on the base image
if [ "$BRANCH_NAME" == "main" ] || [ "$BRANCH_NAME" == "master" ]; then
    docker manifest create ghcr.io/jrcichra/email-fanout:latest \
        --amend ghcr.io/jrcichra/email-fanout:${SHA}-amd64 \
        --amend ghcr.io/jrcichra/email-fanout:${SHA}-arm64
    docker manifest push ghcr.io/jrcichra/email-fanout:latest
fi
