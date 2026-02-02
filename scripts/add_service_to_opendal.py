import sys
import re

def update_pyproject(service_name_raw):
    service_name = service_name_raw.replace("_", "-")
    path = "opendal/pyproject.toml"
    with open(path, "r") as f:
        content = f.read()

    # Uncomment headers if they are commented
    content = content.replace("# [project.optional-dependencies]", "[project.optional-dependencies]")
    content = content.replace("# [tool.uv.sources]", "[tool.uv.sources]")
    
    # 1. Add to optional-dependencies
    opt_deps_match = re.search(r"(?P<header>\[project\.optional-dependencies\])\n(?P<content>.*?)(?=\n\n|\n\[|$)", content, re.DOTALL)
    if opt_deps_match:
        header = opt_deps_match.group("header")
        section_content = opt_deps_match.group("content")
        # Check if already exists
        if f'service-{service_name}' not in section_content:
            # Uncomment '# # Services' if it's there
            section_content = section_content.replace("# # Services", "# Services")
            # Find the last service- line or the # Services line
            lines = section_content.splitlines()
            inserted = False
            for i in range(len(lines) - 1, -1, -1):
                if lines[i].strip().startswith("service-") or lines[i].strip().startswith("# service-"):
                    # Uncomment if it was commented
                    if lines[i].startswith("# "):
                        lines[i] = lines[i][2:]
                    
                    lines.insert(i + 1, f'service-{service_name} = ["opendal-service-{service_name}"]')
                    inserted = True
                    break
            
            if not inserted:
                 # Fallback to appending after the header
                 lines.insert(1, f'service-{service_name} = ["opendal-service-{service_name}"]')
            
            new_section = header + "\n" + "\n".join(lines)
            content = content.replace(opt_deps_match.group(0), new_section)

    # 2. Add to tool.uv.sources
    uv_sources_match = re.search(r"(?P<header>\[tool\.uv\.sources\])\n(?P<content>.*?)(?=\n\n|\n\[|$)", content, re.DOTALL)
    if uv_sources_match:
        header = uv_sources_match.group("header")
        section_content = uv_sources_match.group("content")
        if f'opendal-service-{service_name}' not in section_content:
             section_content = section_content.replace("# # Services", "# Services")
             lines = section_content.splitlines()
             inserted = False
             for i in range(len(lines) - 1, -1, -1):
                if lines[i].strip().startswith("opendal-service-") or lines[i].strip().startswith("# opendal-service-"):
                    if lines[i].startswith("# "):
                        lines[i] = lines[i][2:]
                    lines.insert(i + 1, f'opendal-service-{service_name} = {{ workspace = true }}')
                    inserted = True
                    break
             
             if not inserted:
                 lines.insert(1, f'opendal-service-{service_name} = {{ workspace = true }}')
                 
             new_section = header + "\n" + "\n".join(lines)
             content = content.replace(uv_sources_match.group(0), new_section)

    # Also uncomment existing services in these sections if they are still commented
    content = content.replace("# service-fs", "service-fs")
    content = content.replace("# service-s3", "service-s3")
    content = content.replace("# opendal-service-fs", "opendal-service-fs")
    content = content.replace("# opendal-service-s3", "opendal-service-s3")
    content = content.replace("# # Layers", "# Layers")
    content = content.replace("# layer-retry", "layer-retry")
    content = content.replace("# opendal-layer-retry", "opendal-layer-retry")

    with open(path, "w") as f:
        f.write(content)

if __name__ == "__main__":
    if len(sys.argv) > 1:
        update_pyproject(sys.argv[1])
