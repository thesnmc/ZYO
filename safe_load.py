import time
import math
import multiprocessing

def worker(worker_id):
    while True:
        # Heavy math block
        for i in range(150000):
            x = math.sqrt(i)
        
        # Breathe to keep SSH alive
        time.sleep(0.01)

if __name__ == "__main__":
    cores_to_overwhelm = 4
    swarm_size = 8
    
    print(f"[*] Simulating massive traffic: Spawning {swarm_size} workers across {cores_to_overwhelm} cores...")
    print("[*] SSH Bridge should remain safe. Press Ctrl+C to stop.")
    
    processes = []
    for i in range(swarm_size):
        p = multiprocessing.Process(target=worker, args=(i,))
        p.start()
        processes.append(p)
    
    try:
        for p in processes:
            p.join()
    except KeyboardInterrupt:
        print("\n[*] Recalling the swarm.")
        for p in processes:
            p.terminate()