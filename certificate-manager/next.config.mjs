/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",
  basePath: "/certificate-inventory",
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
};

export default nextConfig;