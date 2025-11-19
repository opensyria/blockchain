#!/bin/bash
# Generate systemd service file for OpenSyria node
# توليد ملف خدمة systemd لعقدة OpenSyria

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
USER=$(whoami)
WORKING_DIR="$HOME/.opensyria/node"
BINARY_PATH=$(which opensyria-node || echo "/usr/local/bin/opensyria-node")
CONFIG_PATH="$HOME/.opensyria/config.toml"

echo -e "${GREEN}🔧 OpenSyria Node Systemd Service Generator${NC}"
echo -e "${GREEN}مولد خدمة systemd لعقدة OpenSyria${NC}"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Warning: Running as root | تحذير: التشغيل كجذر${NC}"
    echo -e "${YELLOW}   Consider running as a non-root user | فكر في التشغيل كمستخدم غير جذر${NC}"
    echo ""
fi

# Interactive configuration
read -p "User to run service as [$USER]: " INPUT_USER
USER=${INPUT_USER:-$USER}

read -p "Working directory [$WORKING_DIR]: " INPUT_DIR
WORKING_DIR=${INPUT_DIR:-$WORKING_DIR}

read -p "Binary path [$BINARY_PATH]: " INPUT_BINARY
BINARY_PATH=${INPUT_BINARY:-$BINARY_PATH}

read -p "Config file path [$CONFIG_PATH]: " INPUT_CONFIG
CONFIG_PATH=${INPUT_CONFIG:-$CONFIG_PATH}

# Verify binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}✗ Binary not found: $BINARY_PATH${NC}"
    echo -e "${RED}✗ الملف التنفيذي غير موجود${NC}"
    echo ""
    echo "Build and install the node first:"
    echo "  cargo build --release -p opensyria-node-cli"
    echo "  sudo cp target/release/opensyria-node /usr/local/bin/"
    exit 1
fi

# Create working directory
mkdir -p "$WORKING_DIR"

# Generate service file
SERVICE_FILE="opensyria-node.service"

cat > "$SERVICE_FILE" << EOF
[Unit]
Description=OpenSyria Blockchain Node
Description[ar]=عقدة بلوكتشين OpenSyria
Documentation=https://github.com/opensyria/blockchain
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$WORKING_DIR
ExecStart=$BINARY_PATH daemon --config $CONFIG_PATH
Restart=on-failure
RestartSec=10s

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=$WORKING_DIR

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=opensyria-node

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Automatic log rotation (systemd handles this)
# التناوب التلقائي للسجلات (يتعامل معه systemd)

[Install]
WantedBy=multi-user.target
EOF

echo ""
echo -e "${GREEN}✓ Service file generated: $SERVICE_FILE${NC}"
echo -e "${GREEN}✓ تم إنشاء ملف الخدمة${NC}"
echo ""

# Show installation instructions
echo -e "${YELLOW}Installation steps | خطوات التثبيت:${NC}"
echo ""
echo "1. Copy service file | نسخ ملف الخدمة:"
echo "   sudo cp $SERVICE_FILE /etc/systemd/system/"
echo ""
echo "2. Reload systemd | إعادة تحميل systemd:"
echo "   sudo systemctl daemon-reload"
echo ""
echo "3. Enable service (start on boot) | تفعيل الخدمة (البدء عند التشغيل):"
echo "   sudo systemctl enable opensyria-node"
echo ""
echo "4. Start service | بدء الخدمة:"
echo "   sudo systemctl start opensyria-node"
echo ""
echo "5. Check status | التحقق من الحالة:"
echo "   sudo systemctl status opensyria-node"
echo ""
echo "6. View logs | عرض السجلات:"
echo "   sudo journalctl -u opensyria-node -f"
echo ""

# Optionally install immediately
read -p "Install service now? (y/N): " INSTALL
if [ "$INSTALL" = "y" ] || [ "$INSTALL" = "Y" ]; then
    echo ""
    echo -e "${YELLOW}Installing service...${NC}"
    
    sudo cp "$SERVICE_FILE" /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable opensyria-node
    
    echo ""
    echo -e "${GREEN}✓ Service installed and enabled${NC}"
    echo -e "${GREEN}✓ تم تثبيت وتفعيل الخدمة${NC}"
    echo ""
    
    read -p "Start service now? (y/N): " START
    if [ "$START" = "y" ] || [ "$START" = "Y" ]; then
        sudo systemctl start opensyria-node
        sleep 2
        sudo systemctl status opensyria-node
    fi
fi

echo ""
echo -e "${GREEN}🚀 Done! | تم!${NC}"
