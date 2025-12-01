#!/bin/bash

# Test Ubuntu
docker build -f scripts/dockerfiles/test-ubuntu-packaging.dockerfile -t pg-sqlgen-test-ubuntu .

# Test Debian
docker build -f scripts/dockerfiles/test-debian-packaging.dockerfile -t pg-sqlgen-test-debian .

# Test Fedora
#docker build -f scripts/dockerfiles/test-fedora-packaging.dockerfile -t pg-sqlgen-test-fedora .