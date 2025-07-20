#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
SERVER_IP="178.156.174.75"
SERVER_USER="root"
REMOTE_DIR="/opt/boid-wars"
IMAGE_NAME="boid-wars"

echo -e "${BLUE}🚀 Deploying Boid Wars to Hetzner server...${NC}"

# Create source directory on server if it doesn't exist
echo -e "${YELLOW}📁 Creating source directory on server...${NC}"
ssh ${SERVER_USER}@${SERVER_IP} "mkdir -p ${REMOTE_DIR}/source"

# Sync source code to server (excluding unnecessary files)
echo -e "${YELLOW}📤 Syncing source code to server...${NC}"
rsync -avz --delete \
    --exclude 'target/' \
    --exclude '.git/' \
    --exclude 'node_modules/' \
    --exclude '*.tar.gz' \
    --exclude 'bevy-client/pkg/' \
    --exclude 'lightyear-wasm/pkg/' \
    --exclude '.DS_Store' \
    . ${SERVER_USER}@${SERVER_IP}:${REMOTE_DIR}/source/

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Source code synced successfully${NC}"
else
    echo -e "${RED}❌ Failed to sync source code${NC}"
    exit 1
fi

# Deploy on server
echo -e "${YELLOW}🚀 Deploying on server...${NC}"

ssh ${SERVER_USER}@${SERVER_IP} << 'ENDSSH'
set -e
cd /opt/boid-wars

echo "🏗️  Building Docker image on server (native x86_64)..."
cd source
docker build --no-cache -t boid-wars:latest .
cd ..

echo "🛑 Stopping existing containers..."
docker-compose down || true

echo "🚀 Starting new containers..."
docker-compose up -d

echo "📊 Checking status..."
docker-compose ps
echo ""
echo "📜 Recent logs:"
docker-compose logs --tail=30

echo ""
echo "🔍 Checking if game server started successfully..."
docker-compose logs | grep -E "(Starting game server|error|Error)" | tail -10
ENDSSH

echo -e "${GREEN}✅ Deployment complete!${NC}"
echo -e "${BLUE}🌐 Your game should be available at:${NC}"
echo -e "${BLUE}   - http://${SERVER_IP}/${NC}"
echo -e "${BLUE}   - http://${SERVER_IP}/demo.html${NC}"
echo -e ""
echo -e "${YELLOW}📝 Useful commands:${NC}"
echo -e "   SSH to server: ssh ${SERVER_USER}@${SERVER_IP}"
echo -e "   View logs: ssh ${SERVER_USER}@${SERVER_IP} 'cd ${REMOTE_DIR} && docker-compose logs -f'"
echo -e "   Restart: ssh ${SERVER_USER}@${SERVER_IP} 'cd ${REMOTE_DIR} && docker-compose restart'"