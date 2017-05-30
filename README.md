# MultiStagger

> This project started off as a little bit of a joke...
> Nothing has changed.

#### POC: Not actually working yet

MultiStagger allows you to run Docker multi-stage `Dockerfile`s without having
to actually update docker. Simply call `multistagger` infront of your docker
command.

Your `Dockerfile` should have multiple stages for you to get the most benefit
out of it:

```Dockerfile
FROM alpine:latest as first

RUN echo "foo" > /file.txt

FROM alpine:latest as second

COPY --from=first /file.txt /test.txt

RUN cat /test.txt
```

This is useful if you are working somewhere that can't update docker to the
latest but you want to have a forwards compatible way of implementing multistage
builds today.

