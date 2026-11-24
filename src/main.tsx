import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import App from "./App";
import DeviceWindow from "./DeviceWindow";

function Root() {
  const params = new URLSearchParams(window.location.search);
  if (params.get("view") === "device") {
    return (
      <DeviceWindow
        vid={params.get("vid") ?? "N/A"}
        pid={params.get("pid") ?? "N/A"}
        name={params.get("name") ?? "未知设备"}
      />
    );
  }
  return <App />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider defaultColorScheme="dark">
      <Root />
    </MantineProvider>
  </React.StrictMode>,
);
