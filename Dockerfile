FROM rust:1.69
MAINTAINER Ugur Cayoglu <cayoglu@me.com>

ADD ./head-scratcher /source/head-scratcher
ADD ./visualize /source/visualize
ADD ./build.nre.sh /source/

WORKDIR /source
RUN ./build.nre.sh

FROM node:20
RUN mkdir /root/netcdf
ADD ./earth /root/netcdf/earth
COPY --from=0 /source/visualize/pkg /root/netcdf/earth/public/pkg
RUN cd /root/netcdf/earth && npm ci
RUN cd /root/netcdf/earth && ./node_modules/.bin/vite build

FROM nginx:alpine
RUN rm -rf /usr/share/nginx/html/*
COPY --from=1 /root/netcdf/earth/dist /usr/share/nginx/html

CMD ["nginx", "-g", "daemon off;"]
