# Authy
A rust authentication server


## Speed tests
Speed tests done on a terrible 6 year old 4 core laptop. I can only assume if someone uses
this on real hardware that isn't overheating on a nearly dead battery you may get better results.

Also, any endpoint that requires processing a password will be very slow by default because
the password library (Argon2) uses the difficulty of computing its password hashes as a form
of protection. It can be reconfigured in the argon config section.

### User creation
You can create over 300 new users per second
```
Running 20s test @ http://localhost:8000/create
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    58.24ms   18.76ms 150.77ms   71.47%
    Req/Sec    17.16      5.96    40.00     56.91%
  6870 requests in 20.04s, 1.17MB read
  Non-2xx or 3xx responses: 6870
Requests/sec:    342.89
Transfer/sec:     59.60KB
```
### Login
You can have 330 people log in FOR THE FIRST TIME. Subsequent logins should use the
refresh token and not affect this limit.
```
Running 20s test @ http://localhost:8000/login
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    60.42ms   19.54ms 157.50ms   70.99%
    Req/Sec    16.54      5.91    30.00     53.54%
  6622 requests in 20.05s, 3.25MB read
Requests/sec:    330.29
Transfer/sec:    165.79KB
```

### Re-Login / Refresh
You can have over 60k people get a new access / refresh token every second. If you're
using that anywhere near this level of requests please go to firebase or a solution not
written by someone learning rust.
```
Running 20s test @ http://localhost:8000/refresh
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   350.20us  253.09us  11.00ms   85.52%
    Req/Sec     3.08k   388.97     4.34k    68.05%
  1228523 requests in 20.10s, 637.36MB read
Requests/sec:  61120.47
Transfer/sec:     31.71MB
```

### Access Request
You can validate almost 60k access requests in a single second. Again, if you're anywhere near
this you should be using something that has a non-hobby developer behind it.
```
Running 20s test @ http://localhost:8000/access
  20 threads and 20 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   358.18us  246.14us   8.28ms   83.61%
    Req/Sec     2.95k   369.95     3.99k    66.85%
  1178776 requests in 20.10s, 131.53MB read
Requests/sec:  58646.90
Transfer/sec:      6.54MB
```
