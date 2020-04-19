From alpine

RUN apk --update add && apk add gnuplot curl font-noto
RUN curl -L https://github.com/sile/kurobako/releases/download/0.1.10/kurobako-0.1.10.linux-amd64 -o kurobako && chmod +x kurobako && mv kurobako /usr/local/bin/
ENTRYPOINT ["kurobako"]
