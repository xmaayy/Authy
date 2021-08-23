# Testing user creation
wrk -t20 -c20 -d20s -s ./post_wrk.lua http://localhost:8000/create
# Testing user listing
wrk -t20 -c20 -d20s  http://localhost:8000/list
