FROM docker.io/library/ubuntu:22.04

ARG PROFILE=debug

RUN apt update -y \
  && apt install -y ca-certificates libssl-dev tzdata

WORKDIR /l2o

COPY . /l2o
COPY ./target/${PROFILE}/l2o-cli /l2o

RUN echo '#!/bin/bash\n/l2o/l2o-cli $@' > /l2o/.entrypoint.sh
RUN chmod u+x /l2o/.entrypoint.sh

ENTRYPOINT ["/l2o/.entrypoint.sh"]
