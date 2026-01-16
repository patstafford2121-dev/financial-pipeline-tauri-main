#!/usr/bin/env python3
"""
Setup Script - Financial Data Pipeline
Door 865: Financial Data Pipeline - SQLite Edition
"""

import os
import sys
import json
import subprocess
from pathlib import Path


def print_header(text):
    print("\n" + "=" * 60)
    print(text)
    print("=" * 60)


def check_python_version():
    """Ensure Python 3.8+"""
    if sys.version_info < (3, 8):
        print("❌ Python 3.8+ required")
        print(f"   Current: {sys.version}")
        return False
    print(f"✓ Python {sys.version_info.major}.{sys.version_info.minor}")
    return True


def install_dependencies():
    """Install required packages"""
    print("\nInstalling dependencies...")
    try:
        subprocess.run([sys.executable, "-m", "pip", "install", "-r", "requirements.txt"], 
                      check=True, capture_output=True)
        print("✓ Dependencies installed")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        return False


def create_config():
    """Create config from template"""
    config_path = "config/config.json"
    example_path = "config/config.example.json"
    
    if os.path.exists(config_path):
        print(f"✓ Config already exists at {config_path}")
        return True
    
    if not os.path.exists(example_path):
        print(f"❌ Template not found: {example_path}")
        return False
    
    # Copy template
    import shutil
    shutil.copy(example_path, config_path)
    print(f"✓ Created config at {config_path}")
    print("  ⚠ IMPORTANT: Edit config/config.json and add your API keys!")
    return True


def create_directories():
    """Create necessary directories"""
    dirs = [
        "data",
        "data/raw",
        "data/backups",
        "logs"
    ]
    
    for d in dirs:
        os.makedirs(d, exist_ok=True)
    
    print(f"✓ Created {len(dirs)} directories")
    return True


def init_database():
    """Initialize database schema"""
    print("\nInitializing database...")
    try:
        subprocess.run([sys.executable, "scripts/init_db.py"], check=True)
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Database initialization failed: {e}")
        return False


def prompt_api_keys():
    """Interactive API key setup"""
    print_header("API Key Configuration (Optional)")
    print("\nYou can add API keys now or edit config/config.json later.")
    print("Free tier keys available at:")
    print("  - Alpha Vantage: https://www.alphavantage.co/support/#api-key")
    print("  - Finnhub: https://finnhub.io/register")
    print("  - FMP: https://site.financialmodelingprep.com/developer/docs")
    
    response = input("\nConfigure API keys now? (y/N): ")
    
    if response.lower() != 'y':
        print("Skipping API key setup. Edit config/config.json manually.")
        return True
    
    # Load config
    with open("config/config.json", 'r') as f:
        config = json.load(f)
    
    # Prompt for keys
    keys = {}
    
    alpha = input("\nAlpha Vantage API key (press Enter to skip): ").strip()
    if alpha:
        keys['alpha_vantage'] = alpha
    
    finn = input("Finnhub API key (press Enter to skip): ").strip()
    if finn:
        keys['finnhub'] = finn
    
    fmp = input("FMP API key (press Enter to skip): ").strip()
    if fmp:
        keys['fmp'] = fmp
    
    # Update config
    if keys:
        config['api_keys'].update(keys)
        with open("config/config.json", 'w') as f:
            json.dump(config, f, indent=2)
        print(f"\n✓ Saved {len(keys)} API keys to config")
    
    return True


def show_next_steps():
    """Show what to do next"""
    print_header("Setup Complete!")
    
    print("\n✓ Database initialized")
    print("✓ Config created")
    print("✓ Ready to load data")
    
    print("\nNext steps:")
    print("\n1. Load symbol catalog:")
    print("   python scripts/load_symbols.py --download --asset-class equity")
    
    print("\n2. Fetch price data:")
    print("   python scripts/fetch_prices.py --symbols AAPL,MSFT,GOOGL")
    
    print("\n3. Load macro data:")
    print("   python scripts/fetch_macro.py")
    
    print("\n4. Query your data:")
    print("   python scripts/query.py --sector")
    
    print("\nDocumentation:")
    print("  - Quick start: docs/QUICKSTART.md")
    print("  - PhiSHRI Door 865 for full reference")
    
    print("\n" + "=" * 60)


def main():
    print_header("Financial Data Pipeline - Setup")
    print("Door 865: Financial Data Pipeline - SQLite Edition")
    
    # Check Python version
    if not check_python_version():
        return 1
    
    # Install dependencies
    if not install_dependencies():
        return 1
    
    # Create config
    if not create_config():
        return 1
    
    # Create directories
    if not create_directories():
        return 1
    
    # Initialize database
    if not init_database():
        return 1
    
    # Optional: API key setup
    prompt_api_keys()
    
    # Show next steps
    show_next_steps()
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
