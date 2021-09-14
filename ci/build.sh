#!/bin/sh
# -t tags the image
docker build -t rustspot -f ci/Dockerfile .
