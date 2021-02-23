## Weather reports
Small app that grabs reports from two weather providers and returns average report for specified location.

### Setup

Clone rerpo

```
git clone git@github.com:daniilsunyaev/servers_ping_stats.git
```

App requires weather api providers' api keys to run.

```
OPEN_WEATHER_APPID=1 WEATHERBIT_API_KEY=2 cargo run
```

You can use .env files to set up env vars:

```
mv .env.example .env
```

Open `.env` and set both `WEATHERBIT_API_KEY` and `OPEN_WEATHER_APPID`.

Note that intertation tests are making actual requests to weather api providers, so you need to set up api keys before running `cargo test`.

### Usage

get current weather:
```
curl "localhost:7878/daily?city_name=moscow"
Tue Feb 23, temperature: -17.66
```

get weather forecast for specific day:
```
curl "localhost:7878/daily?city_name=moscow&days_since=2"
Wed Feb 24, temperature: -12.135
```

get weather forecast for 5 days:
```
curl "localhost:7878/forecast?city_name=london"
Tue Feb 23, temperature: 11.515
Tue Feb 23, temperature: 11.865
Wed Feb 24, temperature: 8.775
Thu Feb 25, temperature: 8.76
Thu Feb 25, temperature: 9.775
```
