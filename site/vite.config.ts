import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    watch: {
      // The repo lives on a Windows-mounted drive (/mnt/e) where inotify
      // events never arrive in WSL; without polling Vite serves stale code.
      usePolling: true,
      interval: 400,
    },
  },
});
