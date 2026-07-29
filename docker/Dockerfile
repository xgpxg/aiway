FROM alpine:3.20

RUN apk add --no-cache ca-certificates tzdata

COPY build/aiway /usr/local/bin/aiway

EXPOSE 7000 7001 7080 7443

ENTRYPOINT ["aiway"]
CMD ["--access-address", "0.0.0.0", "--console-address", "0.0.0.0", "--gateway-address", "0.0.0.0"]
