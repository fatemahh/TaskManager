# TaskManager
Task Manager in Rust for Linux
Milestone 1 Objectives:
- View processes ID					DONE
- View processes CPU usage & Memory usage					DONE
- View process status					DONE
- Sort process using most CPU usage					DONE
- Sort process using most Memory usage					DONE
- Kill process using PID					DONE
- Sleep  process using PID					DONE
- Resume process using PID					DONE
- Search process using PID					DONE
- Show total number of processes, sleeping processes, running processes, stopped processes			DONE

Later Objectives:
- GUI					DONE
- Tree view with parent & children processes					DONE
- Sort processes by CPU usage or memory usage					DONE
- Send notification if the process reaches a pre-set max for CPU or memory usage					DONE
- Filter processes using status					DONE
- Color-coded					DONE
- CPU and Memory real-time graph of utilization

Journal:
- 06/11/24: Fatemah created repo on github.
- 06/11/24: Amany added the view of CPU and Memory for each process.
- 07/11/24: Yussuf added real-time view of CPU and Memory for each process, sorting processes using Memory then CPU.
- 11/11/24: Fatemah changed from 1 second to 0.1 second to refresh, show parent id of each process, show status of process.
- 13/11/24: Amany added kill proccess, resume proccess, sleep proccess using PID to main2.rs.
- 13/11/24: Yussuf added the start screen, asking to display usage or exit.
- 14/11/24: Yussuf merged main.rs and main2.rs, also added help command which shows all possible commands.
- 14/11/24: Mariam implemented searching for a process by PID and displaying its details.
- 15/11/24: Mariam implemented functionality to show the total number of processes, as well as the number of sleeping, running, and stopped processes.
- 16/11/24: Fatemah added filter processes using status functionality.
- 24/11/24: Yussuf added EGUI basic skeleton which displays processes and their data with some color coding.
- 25/11/24: Fatemah changed directory to have cmd and gui, trying to change speed of gui change.
- 26/11/24: Yussuf added color coding to CPU, added sort by CPU and Memory (ASC and DESC) and improved process display.
- 26/11/24: Fatemah combined the GUI and CMD in TaskManagerGUI, you can now open both in TaskManagerGUI, also rearranged the code for clarity.
- 28/11/24: Amany implemented the tree view with parent and child processes in separate folder.
- 28/11/24: Fatemah combined tree view with full code, implemented sending notification if process reaches max cpu or mem usage.
- 29/11/24: Yussuf added sticky headers, improved Tree view look, and improved notifications look.
- 30/11/24: Fatemah did a final test before demo.
