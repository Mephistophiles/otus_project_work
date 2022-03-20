FROM node:alpine

COPY . /src

RUN cd /src \
	&& yarn \
	&& yarn build


FROM nginx:stable-alpine

COPY --from=0 /src/dist /srv/barrier
COPY nginx_config/nginx.conf /etc/nginx/nginx.conf
COPY nginx_config/default.conf /etc/nginx/conf.d/default.conf

WORKDIR /srv/barrier/

RUN chown nginx:nginx /srv/barrier

CMD ["nginx", "-g", "daemon off;"]
