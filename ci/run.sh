#!/bin/bash
# Create a volume for target?
# -it creates an interactive bash shell in the container
# -rm removes container when it exits
docker run --platform=linux/arm64 --rm --ipc=host \
    --mount type=bind,source="$(pwd)/target/arm64",target=/output \
    rustspot \
    "cp -r target/* /output"