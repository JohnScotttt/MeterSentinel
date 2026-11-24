import { Badge, Box, Group, Stack, Text, Title } from "@mantine/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

interface DeviceWindowProps {
  vid: string;
  pid: string;
  name: string;
}

const DOT_COLORS = {
  close: "#ff5f57",
  minimize: "#febc2e",
  zoom: "#28c840",
};

function DeviceWindow({ vid, pid, name }: DeviceWindowProps) {
  const appWindow = getCurrentWindow();

  // 内容渲染完成后再显示窗口，避免启动白屏
  useEffect(() => {
    const raf = requestAnimationFrame(() => {
      void appWindow.show();
    });
    return () => cancelAnimationFrame(raf);
  }, [appWindow]);

  return (
    <Stack gap={0} style={{ height: "100vh" }}>
      {/* 弱化的标题栏：整条可拖拽，仅左上角三圆 */}
      <Box
        px="sm"
        style={{
          height: 32,
          flexShrink: 0,
          display: "flex",
          alignItems: "center",
          userSelect: "none",
        }}
        data-tauri-drag-region
      >
        <Group gap={8} style={{ pointerEvents: "auto" }}>
          <Box
            component="button"
            aria-label="关闭"
            onClick={() => appWindow.close()}
            style={{
              width: 12,
              height: 12,
              padding: 0,
              borderRadius: "50%",
              border: "none",
              cursor: "pointer",
              backgroundColor: DOT_COLORS.close,
            }}
          />
          <Box
            aria-hidden
            style={{
              width: 12,
              height: 12,
              borderRadius: "50%",
              backgroundColor: DOT_COLORS.minimize,
              opacity: 0.5,
            }}
          />
          <Box
            aria-hidden
            style={{
              width: 12,
              height: 12,
              borderRadius: "50%",
              backgroundColor: DOT_COLORS.zoom,
              opacity: 0.5,
            }}
          />
        </Group>
      </Box>

      {/* 内容区 */}
      <Stack
        align="center"
        justify="center"
        gap="md"
        style={{ flex: 1, padding: 24, paddingTop: 8 }}
      >
        <Title order={4}>{name}</Title>
        <Group gap="lg">
          <Stack gap={4} align="center">
            <Text size="sm" c="dimmed">
              VID
            </Text>
            <Badge size="lg" variant="light">
              {vid}
            </Badge>
          </Stack>
          <Stack gap={4} align="center">
            <Text size="sm" c="dimmed">
              PID
            </Text>
            <Badge size="lg" variant="light" color="grape">
              {pid}
            </Badge>
          </Stack>
        </Group>
      </Stack>
    </Stack>
  );
}

export default DeviceWindow;
