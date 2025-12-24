#!/bin/bash
../../target/otp_root/bin/beam &
BEAM_PID=$!
echo "BEAM PID: $BEAM_PID"
sleep 2
echo "JIT code should be allocated now. Press Ctrl+C to continue..."
trap "kill $BEAM_PID 2>/dev/null" EXIT
wait $BEAM_PID
