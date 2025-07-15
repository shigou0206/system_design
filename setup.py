#!/usr/bin/env python3
"""
Setup script for TRN (Tool Resource Name) Python Library
"""

from setuptools import setup, find_packages
import os

# Read the README file
def read_readme():
    """Read the README.md file for long description."""
    here = os.path.abspath(os.path.dirname(__file__))
    with open(os.path.join(here, 'README.md'), encoding='utf-8') as f:
        return f.read()

# Read version from package
def read_version():
    """Read version from the package."""
    here = os.path.abspath(os.path.dirname(__file__))
    version_file = os.path.join(here, 'trn', '__init__.py')
    
    version = None
    with open(version_file, 'r', encoding='utf-8') as f:
        for line in f:
            if line.startswith('__version__'):
                # Extract version from line like: __version__ = "1.0.0"
                version = line.split('=')[1].strip().strip('"').strip("'")
                break
    
    if version is None:
        raise RuntimeError('Cannot find version information')
    
    return version

setup(
    name="trn-library",
    version=read_version(),
    author="AI Platform Team",
    author_email="team@aiplatform.com",
    description="Enterprise-grade Python library for Tool Resource Names (TRN)",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/your-org/trn-library",
    project_urls={
        "Bug Reports": "https://github.com/your-org/trn-library/issues",
        "Source": "https://github.com/your-org/trn-library",
        "Documentation": "https://github.com/your-org/trn-library/wiki",
    },
    
    # Package information
    packages=find_packages(exclude=['tests*', 'examples*']),
    python_requires=">=3.7",
    
    # Dependencies
    install_requires=[
        # Core dependencies are minimal - using only Python standard library
    ],
    
    # Optional dependencies
    extras_require={
        'dev': [
            'pytest>=6.0',
            'pytest-cov>=2.0',
            'black>=21.0',
            'isort>=5.0',
            'flake8>=3.8',
            'mypy>=0.800',
        ],
        'docs': [
            'sphinx>=4.0',
            'sphinx-rtd-theme>=0.5',
        ],
        'performance': [
            'cachetools>=4.0',  # Advanced caching
            'rapidfuzz>=1.0',   # Fast string matching
        ]
    },
    
    # Package metadata
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: System :: Systems Administration",
        "Topic :: Internet :: WWW/HTTP :: Dynamic Content",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Operating System :: OS Independent",
        "Typing :: Typed",
    ],
    
    keywords="trn tool resource name identifier ai agent platform parsing validation",
    
    # Entry points for command line tools (if needed)
    entry_points={
        'console_scripts': [
            'trn-validate=trn.cli:validate_command',
            'trn-parse=trn.cli:parse_command',
        ],
    },
    
    # Include additional files
    include_package_data=True,
    package_data={
        'trn': ['py.typed'],  # Type hints marker file
    },
    
    # Zip safety
    zip_safe=False,
    
    # Test suite
    test_suite='tests',
    tests_require=[
        'pytest>=6.0',
        'pytest-cov>=2.0',
    ],
) 