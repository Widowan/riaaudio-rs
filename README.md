## What is this?

A small rust app that utilizes yt-dlp to download music from configured youtube channels and upload the audio to telegram

Included features:
- Automatic install and update of yt-dlp inside container
- Persists video IDs and song titles to avoid duplicates
- Skips videos with blacklisted patterns ("Best of", "Mix", etc)
- Automatically strips `(Lyrics video)`, `[Monstercat Release]`, `(Official Visualizer)` and the likes from the title ([full list](src/branch/main/src/utils.rs#L147))

## Can I see it live?

https://t.me/RiaAudio

## How do I use this?

A docker container is available in the packages menu:

```shell
docker run \
  -v ./data:/data \
  -e RIA_TELEGRAM_TOKEN=xxxxx:xxxxx \
  -e RIA_TELEGRAM_CHAT_ID=@RiaAudio \
  -e RIA_SLEEP_TIMER=1800 \
  --restart always \
  ghcr.io/widowan/riaaudio-rs:latest
```

Example `config.yml` file:
```yaml
channels:
  - 'UCMOgdURr7d8pOVlc-alkfRg' # xkito
  - 'UCe55Gy-hFDvLZp8C8BZhBnw' # nb3
  - 'UC3ifTl5zKiCAhHIBQYcaTeg' # proximity
  - 'UC0n9yiP-AD2DpuuYCDwlNxQ' # tasty
  - 'UCBvc2GVFfuY6zxqdjNF-6sQ' # arcadium
```

You can build it natively with `cargo build --release`, but it's recommended to run inside container