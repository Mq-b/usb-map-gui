# 开发测试工具

## 虚拟 USB 设备

用于在没有物理硬件时测试设备列表功能。

```bash
# 创建虚拟设备（需要 sudo）
./dev/virtual-usb.sh create

# 查看状态
./dev/virtual-usb.sh status

# 测试完成后删除
./dev/virtual-usb.sh remove
```

重启后自动消失，无需手动清理。

### 创建的设备

| 类型 | 设备 | 说明 |
|------|------|------|
| 物理设备 | `/dev/ttyUSB0` | 虚拟 USB 串口 |
| 物理设备 | `/dev/ttyUSB1` | 虚拟 USB 串口 |
| 物理设备 | `/dev/ttyACM0` | 虚拟 ACM 设备 |
| 符号链接 | `/dev/ttyMyDevice` -> `ttyUSB0` | 模拟已有 udev 规则 |
| 符号链接 | `/dev/ttyModem` -> `ttyACM0` | 模拟已有 udev 规则 |

注意：这些设备没有真实 USB 接口 ID，接口 ID 列会显示 `N/A`。
