#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the app name from fly.toml
APP_NAME=$(grep "^app = " fly.toml | sed "s/app = '\(.*\)'/\1/")

if [ -z "$APP_NAME" ]; then
    echo -e "${RED}❌ Could not find app name in fly.toml${NC}"
    exit 1
fi

echo -e "${BLUE}🚀 Deploying ${APP_NAME} to Fly.io (local build)...${NC}"

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker is not running. Please start Docker Desktop.${NC}"
    exit 1
fi

# Check if user is logged in to Fly.io
if ! flyctl auth whoami &> /dev/null; then
    echo -e "${YELLOW}🔑 Please log in to Fly.io...${NC}"
    flyctl auth login
fi

# Build the Docker image for AMD64 architecture (required for Fly.io)
echo -e "${YELLOW}🏗️  Building Docker image for AMD64 architecture...${NC}"
echo -e "${YELLOW}   This is required for Fly.io deployment (cross-platform build)${NC}"
echo -e "${YELLOW}   This may take several minutes for the first build...${NC}"

# Ensure buildx is available and create/use a builder instance
if ! docker buildx create --use --name fly-builder 2>/dev/null; then
    docker buildx use fly-builder
fi

# Build for linux/amd64 platform explicitly
if docker buildx build --platform linux/amd64 -t ${APP_NAME} --load .; then
    echo -e "${GREEN}✅ Docker build successful (AMD64)${NC}"
else
    echo -e "${RED}❌ Docker build failed${NC}"
    exit 1
fi

# Get image size
IMAGE_SIZE=$(docker images ${APP_NAME} --format "{{.Size}}" | head -1)
echo -e "${BLUE}📦 Image size: ${IMAGE_SIZE}${NC}"

# Authenticate Docker with Fly.io registry
echo -e "${YELLOW}🔑 Authenticating with Fly.io registry...${NC}"
if ! fly auth docker; then
    echo -e "${RED}❌ Failed to authenticate with Fly.io registry${NC}"
    exit 1
fi

# Tag the image for Fly.io registry
echo -e "${YELLOW}🏷️  Tagging image for Fly.io registry...${NC}"
docker tag ${APP_NAME} registry.fly.io/${APP_NAME}

# Push to Fly.io registry
echo -e "${YELLOW}⬆️  Pushing image to Fly.io registry...${NC}"
echo -e "${YELLOW}   This may take a while depending on your internet speed...${NC}"

if docker push registry.fly.io/${APP_NAME}; then
    echo -e "${GREEN}✅ Image pushed successfully${NC}"
else
    echo -e "${RED}❌ Failed to push image to registry${NC}"
    echo -e "${YELLOW}💡 Tip: If this fails, try running 'fly auth docker' again${NC}"
    exit 1
fi

# Deploy the pushed image
echo -e "${YELLOW}🚀 Deploying to Fly.io...${NC}"

if fly deploy --image registry.fly.io/${APP_NAME}; then
    echo -e "${GREEN}✅ Deployment complete!${NC}"
    echo -e "${BLUE}🌐 Your app should be available at: https://${APP_NAME}.fly.dev${NC}"
    
    # Show app status
    echo -e "\n${BLUE}📊 App Status:${NC}"
    fly status
else
    echo -e "${RED}❌ Deployment failed${NC}"
    echo -e "${YELLOW}💡 Check logs with: fly logs${NC}"
    exit 1
fi

# Optional: Show recent logs
echo -e "\n${BLUE}📜 Recent logs:${NC}"
fly logs --limit 20