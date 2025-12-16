#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

AWS_REGION="ap-southeast-2"
AWS_ACCOUNT_ID="968061875204"
ECR_REGISTRY="${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
ECR_REPO="logos-blockchain-testing"
TAG="test"

LOCAL_IMAGE="${ECR_REPO}:${TAG}"
REMOTE_IMAGE="${ECR_REGISTRY}/${ECR_REPO}:${TAG}"

export DOCKER_DEFAULT_PLATFORM="linux/amd64"
export CIRCUITS_PLATFORM="${CIRCUITS_PLATFORM:-linux-x86_64}"
export IMAGE_TAG="${LOCAL_IMAGE}"

"${ROOT_DIR}/testing-framework/assets/stack/scripts/build_test_image.sh"

aws ecr get-login-password --region "${AWS_REGION}" \
  | docker login --username AWS --password-stdin "${ECR_REGISTRY}"

docker tag "${LOCAL_IMAGE}" "${REMOTE_IMAGE}"
docker push "${REMOTE_IMAGE}"

echo "${REMOTE_IMAGE}"
