# Use specific Node.js version
FROM node:18.20.4-alpine AS build

# Create non-root user
RUN addgroup -g 1001 -S nodejs && adduser -S nextjs -u 1001

# Set the working directory
WORKDIR /app

# Copy package files
COPY --chown=nextjs:nodejs package*.json ./

# Install dependencies
RUN npm ci --only=production

# Run audit fix (allow to fail for build to continue)
RUN npm audit fix || true

# Copy source code
COPY --chown=nextjs:nodejs . .

# Set environment variable for Docker build
ENV REACT_APP_API_URL=""

# Build application
RUN npm run build

# Production stage
FROM nginx:1.27.2-alpine AS production

# Create non-root user for nginx
RUN addgroup -g 1001 -S nginx-user && adduser -S nginx-user -u 1001

# Create necessary directories and set permissions
RUN mkdir -p /var/cache/nginx/client_temp /var/cache/nginx/proxy_temp /var/cache/nginx/fastcgi_temp /var/cache/nginx/uwsgi_temp /var/cache/nginx/scgi_temp && \
    chown -R nginx-user:nginx-user /var/cache/nginx && \
    chown -R nginx-user:nginx-user /var/log/nginx && \
    chown -R nginx-user:nginx-user /etc/nginx/conf.d

# Copy built application
COPY --from=build --chown=nginx-user:nginx-user /app/build /usr/share/nginx/html
COPY --chown=nginx-user:nginx-user nginx.conf /etc/nginx/conf.d/default.conf

# Update nginx configuration for security
RUN sed -i 's/user nginx;/user nginx-user;/' /etc/nginx/nginx.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]