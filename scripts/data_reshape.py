import sys
from pathlib import Path
import pandas as pd
from mcap.reader import make_reader
from geometry_msgs.msg import Twist
from sensor_msgs.msg import LaserScan
from rclpy.serialization import deserialize_message


def main():
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        print("path名を引数に加えてください")
        sys.exit(1)
    file = list(Path(path).glob("*.mcap"))
    if len(file) != 1:
        print("fileが1個ではありません")
        sys.exit(1)
    with open(file[0], "rb") as f:
        reader = make_reader(f)
        scan = []
        cmd_vel = []

        for schema, channel, message in reader.iter_messages():
            if channel.topic == "/cmd_vel":
                data = deserialize_message(message.data, Twist)
                log = {"log_time": message.log_time,
                       "linear_x":data.linear.x,
                       "linear_y":data.linear.y,
                       "linear_z":data.linear.z,
                       "angular_x":data.angular.x,
                       "angular_y":data.angular.y,
                       "angular_z":data.angular.z,}
                cmd_vel.append(log)
            elif channel.topic =="/scan":
                data_scan = deserialize_message(message.data, LaserScan)
                range_dict = {f"r{i}":beam for i,beam in enumerate(data_scan.ranges)}
                stamp_sec = data_scan.header.stamp.sec
                stamp_nanosec = data_scan.header.stamp.nanosec
                log = {}
                log["header_stamp"] = stamp_sec *10**9 + stamp_nanosec
                log["log_time"] =  message.log_time
                log.update(range_dict)
                scan.append(log)
        df_cmd_vel = pd.DataFrame(cmd_vel)
        df_scan = pd.DataFrame(scan)

        output_cmd_vel = f"{path}/data_cmd_vel.csv"
        df_cmd_vel.to_csv(output_cmd_vel, index=False)
        output_scan = f"{path}/data_scan.csv"
        df_scan.to_csv(output_scan, index=False)

if __name__ == "__main__":
    main()