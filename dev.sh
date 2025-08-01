#!/bin/bash

# Judicia Platform Development Server

echo "🚀 Starting Judicia Platform..."

# Function to handle cleanup
cleanup() {
    echo "🛑 Shutting down servers..."
    kill $(jobs -p) 2>/dev/null
    exit
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Start backend
echo "📦 Starting Rust backend..."
cd backend && cargo run &
BACKEND_PID=$!

# Wait a bit for backend to start
sleep 3

# Start frontend
echo "🌐 Starting React frontend..."
cd ../frontend && pnpm dev &
FRONTEND_PID=$!

echo "✅ Services started:"
echo "   🔧 Backend API: http://localhost:8080"
echo "   🌐 Frontend: http://localhost:5173"
echo "   📚 API Docs: http://localhost:8080/health"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for background processes
wait
