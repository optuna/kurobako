From alpine

RUN apk --update add && apk add gnuplot curl font-noto
RUN curl -L https://github.com/sile/kurobako/releases/download/${KUROBAKO_VERSION}/kurobako-${KUROBAKO_VERSION}.linux-amd64 -o kurobako && chmod +x kurobako && mv kurobako /usr/local/bin/
ENTRYPOINT ["kurobako"]
