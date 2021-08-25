# Testing user creation
wrk -t20 -c20 -d20s -s ./post_wrk.lua http://localhost:8000/create
# Testing user listing
wrk -t20 -c20 -d20s  http://localhost:8000/list
wrk -t20 -c20 -d20s -s ./post_wrk.lua http://localhost:8000/login
wrk -t20 -c20 -d20s -s ./post_wrk.lua http://localhost:8000/refresh
wrk -t20 -c20 -d20s -s ./wrk_access.lua http://localhost:8000/access
