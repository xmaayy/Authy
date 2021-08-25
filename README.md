# Authy
A rust authentication server


## Speed tests
Speed tests done on a terrible 6 year old 4 core laptop. I can only assume if someone uses
this on real hardware that isn't overheating on a nearly dead battery you may get better results.

Also, any endpoint that requires processing a password will be very slow by default because
the password library (Argon2) uses the difficulty of computing its password hashes as a form
of protection. It can be reconfigured in the argon config section.

### User creation
You can create 10 new users per second
```
Running 20s test @ http://localhost:8000/create
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.27s   366.62ms   1.94s    63.33%
    Req/Sec     0.14      0.34     1.00     86.46%
  192 requests in 20.09s, 33.56KB read
  Socket errors: connect 0, read 0, write 0, timeout 102
  Non-2xx or 3xx responses: 192
Requests/sec:      9.56
Transfer/sec:      1.67KB
```
### Login
You can have 20 people log in FOR THE FIRST TIME. Subsequent logins should use the
refresh token and not affect this limit.
```
Running 20s test @ http://localhost:8000/login
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   929.00ms  322.04ms   1.85s    71.81%
    Req/Sec     0.71      0.75     3.00     86.30%
  416 requests in 20.04s, 208.81KB read
  Socket errors: connect 0, read 0, write 0, timeout 1
Requests/sec:     20.76
Transfer/sec:     10.42KB
```

### Re-Login / Refresh
You can have over 14k people get a new access / refresh token every second. If you're
using that anywhere near this level of requests please go to firebase or a solution not
written by someone learning rust.
```
Running 20s test @ http://localhost:8000/refresh
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.41ms  536.21us  10.78ms   72.60%
    Req/Sec   717.31     63.82     1.85k    90.40%
  285786 requests in 20.10s, 148.27MB read
Requests/sec:  14217.98
Transfer/sec:      7.38MB
```

### Access Request
You can validate >16k access requests in a single second. Again, if you're anywhere near
this you should be using something that has a non-hobby developer behind it.
```
Running 20s test @ http://localhost:8000/access
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.20ms  476.45us  16.70ms   73.54%
    Req/Sec   840.45     62.50     1.91k    82.25%
  335235 requests in 20.10s, 37.41MB read
Requests/sec:  16678.69
Transfer/sec:      1.86MB
```
