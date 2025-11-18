#!/bin/bash
# Test IPFS Integration for Cultural Heritage Content
# Requirements: IPFS daemon running on localhost:5001

echo "================================"
echo "  IPFS Integration Test"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if IPFS is running
echo "Checking IPFS daemon..."
if ! curl -s http://127.0.0.1:5001/api/v0/version > /dev/null 2>&1; then
    echo -e "${RED}✗ IPFS daemon not running${NC}"
    echo ""
    echo "To start IPFS daemon:"
    echo "  ipfs daemon"
    echo ""
    echo "Or install IPFS:"
    echo "  brew install ipfs"
    echo "  ipfs init"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ IPFS daemon running${NC}"
echo ""

# Build the identity CLI
echo "Building identity CLI..."
cargo build -p opensyria-identity --release 2>&1 | grep -E "(Compiling|Finished)"
IDENTITY_CLI="./target/release/identity"

echo ""
echo "Step 1: Create a cultural heritage token"
echo "----------------------------------------"
$IDENTITY_CLI create \
    --id palmyra-ruins \
    --name "Ruins of Palmyra" \
    --name-ar "آثار تدمر" \
    --description "Ancient Semitic city ruins, UNESCO World Heritage Site" \
    --token-type heritage \
    --category ancient \
    --city "Palmyra" \
    --period "1st-3rd Century CE" \
    --tags "unesco,ancient,desert"

echo ""
echo "Step 2: Create sample heritage content"
echo "--------------------------------------"
cat > palmyra-info.txt << 'EOF'
Palmyra: The Pearl of the Desert
=================================

Palmyra was an ancient Semitic city in present-day Homs Governorate, Syria.
Archaeological finds date back to the Neolithic period, and documents first
mention the city in the early second millennium BC.

Palmyra was a vital caravan stop for travellers crossing the Syrian Desert
and was known for protection of trade routes, as it lay between the Roman
Empire and the Parthian Empire.

UNESCO World Heritage Site
Inscription: 1980
Endangered: 2013

Key Features:
- Temple of Bel
- Monumental Arch
- Theatre
- Valley of the Tombs
- Tetrapylon
EOF

echo -e "${GREEN}✓ Created palmyra-info.txt${NC}"

echo ""
echo "Step 3: Upload content to IPFS"
echo "------------------------------"
$IDENTITY_CLI upload \
    --file palmyra-info.txt \
    --token-id palmyra-ruins

echo ""
echo "Step 4: Verify token has IPFS CID"
echo "---------------------------------"
if grep -q "ipfs_cid" palmyra-ruins.json; then
    echo -e "${GREEN}✓ Token linked to IPFS content${NC}"
    CID=$(grep "ipfs_cid" palmyra-ruins.json | cut -d'"' -f4)
    echo "CID: $CID"
else
    echo -e "${RED}✗ Token not linked to IPFS${NC}"
    exit 1
fi

echo ""
echo "Step 5: Retrieve content from IPFS"
echo "----------------------------------"
$IDENTITY_CLI retrieve "$CID" --output palmyra-retrieved.txt

echo ""
echo "Step 6: Verify content integrity"
echo "--------------------------------"
if diff palmyra-info.txt palmyra-retrieved.txt > /dev/null; then
    echo -e "${GREEN}✓ Content retrieved successfully, files match${NC}"
else
    echo -e "${RED}✗ Content mismatch${NC}"
    exit 1
fi

echo ""
echo "Step 7: Test direct IPFS link"
echo "-----------------------------"
echo "Upload another heritage document..."
cat > damascus-umayyad.json << 'EOF'
{
  "name": "Umayyad Mosque",
  "name_ar": "مسجد بني أمية الكبير",
  "location": "Damascus, Syria",
  "built": "705-715 CE",
  "type": "Mosque",
  "significance": "One of the largest and oldest mosques in the world",
  "features": [
    "Minaret of Jesus",
    "Shrine of John the Baptist",
    "Prayer Hall with Byzantine mosaics",
    "Marble courtyard"
  ],
  "unesco_status": "World Heritage Site (Part of Ancient City of Damascus)"
}
EOF

$IDENTITY_CLI upload --file damascus-umayyad.json

echo ""
echo "================================"
echo -e "${GREEN}  All Tests Passed!${NC}"
echo "================================"
echo ""
echo "Cleanup:"
echo "  rm palmyra-*.txt damascus-umayyad.json"
echo "  rm palmyra-ruins.json"
echo ""
echo "IPFS Gateway URLs:"
echo "  http://127.0.0.1:8080/ipfs/$CID"
echo "  https://ipfs.io/ipfs/$CID"
echo ""
