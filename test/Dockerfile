FROM node:7.4-alpine as node

RUN npm --version

COPY index.js /app/index.js

RUN node /app/index.js

# here is a comment


FROM alpine:latest as alpine

    COPY --from=node /app/file.txt /file.txt

CMD ["cat", "/file.txt"]
