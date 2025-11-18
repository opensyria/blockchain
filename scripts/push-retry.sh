#!/bin/zsh
# Auto-retry git push until GitHub recovers

echo "Starting auto-retry push to GitHub..."
echo "Press Ctrl+C to cancel"
echo ""

attempt=1
max_attempts=60

while [ $attempt -le $max_attempts ]; do
  echo "[$attempt/$max_attempts] Attempting push..."
  
  if git push origin main 2>&1; then
    echo ""
    echo "✓ Push successful!"
    exit 0
  else
    echo "Failed. Waiting 30 seconds before retry..."
    sleep 30
    attempt=$((attempt + 1))
  fi
done

echo ""
echo "Maximum attempts reached. GitHub may still be experiencing issues."
echo "Check status: https://www.githubstatus.com"
