FROM python:3.6-slim as base
FROM base as builder

RUN pip install --upgrade pip
RUN apt update
RUN apt install -y git gcc

ADD requirements.txt /

RUN pip install --no-cache -r requirements.txt

RUN find /usr/local/lib/python3.6/ -name '*.c' -delete
RUN find /usr/local/lib/python3.6/ -name '*.pyd' -delete
RUN find /usr/local/lib/python3.6/ -name '*.pxd' -delete
RUN find /usr/local/lib/python3.6/ -name __pycache__ | xargs rm -r


FROM base

COPY --from=builder /usr/local/lib/python3.6/site-packages/ /usr/local/lib/python3.6/site-packages/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgomp.so.1 /usr/lib/x86_64-linux-gnu/libgomp.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgomp.so.1.0.0 /usr/lib/x86_64-linux-gnu/libgomp.so.1.0.0

ADD kurobako /bin/kurobako

CMD ["/bin/kurobako"]
