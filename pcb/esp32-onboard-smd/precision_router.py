import heapq
import math

class PrecisionRouter:
    def __init__(self, width=74.0, height=54.0, res=0.25):
        self.res = res
        self.nx = int(math.ceil(width / res)) + 1
        self.ny = int(math.ceil(height / res)) + 1
        # grid[layer][x][y]: 0 = free, >0 = net_id (core copper), <0 = obstacle (-1 = board edge, <-1 = clearance of net abs(val))
        self.grid = [[[0 for _ in range(self.ny)] for _ in range(self.nx)] for _ in range(2)]
        
        # Block board borders (0.6mm clearance to edge)
        margin = int(math.ceil(0.6 / res))
        for l in range(2):
            for x in range(self.nx):
                for m in range(margin):
                    self.grid[l][x][m] = -1
                    self.grid[l][x][self.ny - 1 - m] = -1
            for y in range(self.ny):
                for m in range(margin):
                    self.grid[l][m][y] = -1
                    self.grid[l][self.nx - 1 - m][y] = -1

        # Block Mounting Holes (5.0mm pad + 0.3mm clearance = radius 2.8mm)
        for hx, hy in [(3.5, 3.5), (70.5, 3.5), (3.5, 50.5), (70.5, 50.5)]:
            self.block_circle(hx, hy, 2.8, layer=None, val=-1)

        # Block Antenna Keepout Zone: X: 16.5 to 37.5, Y: 43.0 to 54.0
        self.block_rect(16.5, 43.0, 37.5, 54.0, layer=None, val=-1)
        
        # Block USB-C J1 shield/mount corridor on B.Cu
        self.block_rect(0.0, 28.0, 8.5, 40.5, layer=1, val=-1)
        
        self.no_via_zones = []

    def to_grid(self, x, y):
        gx = max(0, min(self.nx - 1, int(round(x / self.res))))
        gy = max(0, min(self.ny - 1, int(round(y / self.res))))
        return gx, gy

    def to_coord(self, gx, gy):
        return round(gx * self.res, 3), round(gy * self.res, 3)

    def block_rect(self, x1, y1, x2, y2, layer=None, val=-1):
        gx1, gy1 = self.to_grid(x1, y1)
        gx2, gy2 = self.to_grid(x2, y2)
        layers = [layer] if layer is not None else [0, 1]
        for l in layers:
            for x in range(min(gx1, gx2), max(gx1, gx2) + 1):
                for y in range(min(gy1, gy2), max(gy1, gy2) + 1):
                    self.grid[l][x][y] = val

    def block_circle(self, cx, cy, radius, layer=None, val=-1):
        gx, gy = self.to_grid(cx, cy)
        r_grid = int(math.ceil(radius / self.res))
        layers = [layer] if layer is not None else [0, 1]
        for l in layers:
            for x in range(gx - r_grid, gx + r_grid + 1):
                for y in range(gy - r_grid, gy + r_grid + 1):
                    if 0 <= x < self.nx and 0 <= y < self.ny:
                        if (x - gx)**2 + (y - gy)**2 <= (radius / self.res)**2:
                            self.grid[l][x][y] = val

    def add_pad(self, cx, cy, w, h, layer, net_id, clearance=0.25):
        # layer: 0 for F.Cu, 1 for B.Cu, None for both (THT)
        # Pad core is set to net_id
        # Pad clearance halo is set to -(1000 + net_id)
        clear_val = -(1000 + net_id)
        layers = [layer] if layer is not None else [0, 1]
        if layer is None:
            self.no_via_zones.append((cx, cy, max(w, h)/2.0 + 0.6))
        else:
            self.no_via_zones.append((cx, cy, max(w, h)/2.0 + 0.45))
        
        # Clearance halo
        gx1_c, gy1_c = self.to_grid(cx - w/2 - clearance, cy - h/2 - clearance)
        gx2_c, gy2_c = self.to_grid(cx + w/2 + clearance, cy + h/2 + clearance)
        for l in layers:
            for x in range(gx1_c, gx2_c + 1):
                for y in range(gy1_c, gy2_c + 1):
                    if 0 <= x < self.nx and 0 <= y < self.ny:
                        if self.grid[l][x][y] == 0:
                            self.grid[l][x][y] = clear_val
                            
        # Core pad
        gx1, gy1 = self.to_grid(cx - w/2, cy - h/2)
        gx2, gy2 = self.to_grid(cx + w/2, cy + h/2)
        for l in layers:
            for x in range(gx1, gx2 + 1):
                for y in range(gy1, gy2 + 1):
                    if 0 <= x < self.nx and 0 <= y < self.ny:
                        self.grid[l][x][y] = net_id

    def is_passable(self, x, y, layer, net_id):
        val = self.grid[layer][x][y]
        if val == 0:
            return True
        if val == net_id:
            return True
        if val == -(1000 + net_id):
            return True
        # Any negative val other than -(1000 + net_id) is another net's clearance zone!
        return False

    def can_place_via(self, x, y, net_id, via_radius=0.60):
        coord_x, coord_y = self.to_coord(x, y)
        for nvx, nvy, r in self.no_via_zones:
            if (coord_x - nvx)**2 + (coord_y - nvy)**2 < r**2:
                return False
        # Via must be passable on BOTH layers within via_radius
        r_grid = int(math.ceil(via_radius / self.res))
        for l in range(2):
            for dx in range(-r_grid, r_grid + 1):
                for dy in range(-r_grid, r_grid + 1):
                    if dx*dx + dy*dy <= (via_radius / self.res)**2:
                        nx, ny = x + dx, y + dy
                        if not (0 <= nx < self.nx and 0 <= ny < self.ny):
                            return False
                        if not self.is_passable(nx, ny, l, net_id):
                            return False
        return True

    def mark_trace(self, path, net_id, trace_radius=0.30):
        # Mark path cells with net_id and clearance
        clear_val = -(1000 + net_id)
        r_grid = int(math.ceil(trace_radius / self.res))
        
        # Expand path to all intermediate grid steps if path has jumps
        dense_pts = []
        for i in range(len(path)):
            dense_pts.append(path[i])
            if i > 0 and path[i][2] == path[i-1][2]:
                x1, y1 = path[i-1][0], path[i-1][1]
                x2, y2 = path[i][0], path[i][1]
                gx1, gy1 = self.to_grid(x1, y1) if isinstance(x1, float) else (x1, y1)
                gx2, gy2 = self.to_grid(x2, y2) if isinstance(x2, float) else (x2, y2)
                steps = max(abs(gx2 - gx1), abs(gy2 - gy1))
                for s in range(1, steps):
                    ix = gx1 + int(round(s * (gx2 - gx1) / steps))
                    iy = gy1 + int(round(s * (gy2 - gy1) / steps))
                    dense_pts.append((ix, iy, path[i][2]))

        for pt in dense_pts:
            x, y, l = pt[0], pt[1], pt[2]
            gx, gy = self.to_grid(x, y) if isinstance(x, float) or isinstance(y, float) else (x, y)
            for dx in range(-r_grid, r_grid + 1):
                for dy in range(-r_grid, r_grid + 1):
                    if dx*dx + dy*dy <= (trace_radius / self.res)**2:
                        nx, ny = gx + dx, gy + dy
                        if 0 <= nx < self.nx and 0 <= ny < self.ny:
                            if self.grid[l][nx][ny] == 0:
                                self.grid[l][nx][ny] = clear_val
            self.grid[l][gx][gy] = net_id

    def mark_via(self, x, y, net_id, via_radius=0.55):
        clear_val = -(1000 + net_id)
        r_grid = int(math.ceil(via_radius / self.res))
        gx, gy = self.to_grid(x, y) if isinstance(x, float) or isinstance(y, float) else (x, y)
        for l in range(2):
            for dx in range(-r_grid, r_grid + 1):
                for dy in range(-r_grid, r_grid + 1):
                    if dx*dx + dy*dy <= (via_radius / self.res)**2:
                        nx, ny = gx + dx, gy + dy
                        if 0 <= nx < self.nx and 0 <= ny < self.ny:
                            if self.grid[l][nx][ny] in (0, clear_val):
                                self.grid[l][nx][ny] = clear_val
            self.grid[l][gx][gy] = net_id

    def route(self, p1, p2, net_id, start_layer=0, end_layer=0):
        gx1, gy1 = self.to_grid(p1[0], p1[1])
        gx2, gy2 = self.to_grid(p2[0], p2[1])

        # Priority Queue: (f_score, g_score, x, y, layer, dir)
        pq = []
        dist = {}
        start_layers = [0, 1] if start_layer is None else [start_layer]
        for sl in start_layers:
            heapq.heappush(pq, (0, 0, gx1, gy1, sl, 0))
            dist[(gx1, gy1, sl, 0)] = 0
        parent = {}
        found = False
        end_node = None

        while pq:
            f, g, cx, cy, cl, cdir = heapq.heappop(pq)
            if (cx, cy) == (gx2, gy2):
                if end_layer is None or cl == end_layer:
                    found = True
                    end_node = (cx, cy, cl, cdir)
                    break

            if g > dist.get((cx, cy, cl, cdir), float('inf')):
                continue

            # Movement moves: dx, dy, move_dir
            moves = [(1, 0, 1), (-1, 0, 1), (0, 1, 2), (0, -1, 2)]
            for dx, dy, mdir in moves:
                nx, ny = cx + dx, cy + dy
                if 0 <= nx < self.nx and 0 <= ny < self.ny:
                    if self.is_passable(nx, ny, cl, net_id):
                        cost = 1.0
                        # Layer preferences: F.Cu (layer 0) prefers X (mdir=1)
                        # B.Cu (layer 1) prefers Y (mdir=2)
                        if cl == 0 and mdir == 2: cost += 0.5
                        if cl == 1 and mdir == 1: cost += 0.5
                        # Bend cost
                        if cdir != 0 and mdir != cdir: cost += 0.4
                        ng = g + cost
                        state = (nx, ny, cl, mdir)
                        if ng < dist.get(state, float('inf')):
                            dist[state] = ng
                            parent[state] = (cx, cy, cl, cdir)
                            h = (abs(nx - gx2) + abs(ny - gy2)) * 1.0
                            heapq.heappush(pq, (ng + h, ng, nx, ny, cl, mdir))

            # Via layer switch
            other_l = 1 - cl
            if self.can_place_via(cx, cy, net_id):
                cost = 8.0
                ng = g + cost
                state = (cx, cy, other_l, 0)
                if ng < dist.get(state, float('inf')):
                    dist[state] = ng
                    parent[state] = (cx, cy, cl, cdir)
                    h = (abs(cx - gx2) + abs(cy - gy2)) * 1.0
                    heapq.heappush(pq, (ng + h, ng, cx, cy, other_l, 0))

        if not found:
            return None

        # Reconstruct path
        path = []
        curr = end_node
        while curr:
            path.append((curr[0], curr[1], curr[2]))
            curr = parent.get(curr)
        path.reverse()

        # Mark trace on grid
        self.mark_trace(path, net_id)
        for i in range(1, len(path)):
            if path[i][2] != path[i-1][2]:
                self.mark_via(path[i-1][0], path[i-1][1], net_id)

        return path

print("PrecisionRouter class complete")
