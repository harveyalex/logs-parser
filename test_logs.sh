#!/bin/bash

# Generate sample Heroku logs for testing the parser

echo "Generating sample Heroku logs..."
echo ""

# Function to generate a log line
generate_log() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6N+00:00")
    local source=$1
    local dyno=$2
    local message=$3
    echo "${timestamp} ${source}[${dyno}]: ${message}"
}

# Generate some sample logs with a delay
while true; do
    generate_log "app" "web.1" "Processing request GET /"
    sleep 0.5

    generate_log "heroku" "router" "at=info method=GET path=/ host=example.com request_id=12345 fwd=\"1.2.3.4\" dyno=web.1 connect=1ms service=25ms status=200 bytes=1234"
    sleep 0.3

    generate_log "app" "web.1" "Info: Request completed in 25ms"
    sleep 0.4

    generate_log "app" "worker.3" "Processing job #12345"
    sleep 0.6

    generate_log "app" "web.2" "Warning: High memory usage detected"
    sleep 0.5

    generate_log "app" "worker.1" "Debug: Checking configuration values"
    sleep 0.3

    generate_log "heroku" "web.1" "State changed from starting to up"
    sleep 0.5

    generate_log "app" "web.1" "Error: Connection timeout to database"
    sleep 0.7

    generate_log "app" "web.1" "Info: Retrying connection..."
    sleep 0.4

    generate_log "heroku" "router" "at=error code=H12 desc=\"Request timeout\" method=GET path=/api/slow host=example.com request_id=67890 fwd=\"1.2.3.4\" dyno=web.1 connect=1ms service=30000ms status=503 bytes=0"
    sleep 0.5
done
