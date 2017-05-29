FROM node:7.4-alpine as node

RUN npm --version

COPY index.js /app/index.js

# here is a comment


FROM alpine:latest as alpine

    COPY --from=node /app/file.txt /app/file.txt

CMD ["cat", "/app/file.txt"]
